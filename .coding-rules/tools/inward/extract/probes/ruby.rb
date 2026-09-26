# SPDX-License-Identifier: MIT
# 参照と抜け道を、標準同梱の解析器（Prism）から取る。**解析器を自作しない。**
#
#     ruby ruby.rb <根>
#
# 経路を組み立てて読み込む箇所は、行き先が静的に決まらないので抜け道として返す。
require "json"
require "pathname"
require "prism"

root = ARGV[0]
if root.nil?
  warn "根を渡していない"
  exit 2
end
base = Pathname.new(root).expand_path
edges = []
escapes = []
undecided = []

LOADERS = %i[require require_relative load autoload].freeze

# 経路を点の名前へ直す。**解決しないと、同じ名前が別の層を指す。**
# `require_relative` は書かれた側からの相対、`require` は探索路からの相対である。
def point(base, rel, spec, how)
  return spec if how == "require" && !spec.start_with?(".")

  here = Pathname.new(rel).dirname
  joined = (here + spec).cleanpath
  joined.to_s.delete_prefix("./")
end

Dir.glob("#{base}/**/*.rb").sort.each do |file|
  rel = Pathname.new(file).relative_path_from(base).to_s
  result = Prism.parse_file(file)
  unless result.success?
    undecided << "#{rel} ── 構文として読めない"
    next
  end
  queue = [result.value]
  until queue.empty?
    node = queue.shift
    next unless node.is_a?(Prism::Node)
    if node.is_a?(Prism::CallNode) && LOADERS.include?(node.name)
      arg = node.arguments&.arguments&.first
      line = node.location.start_line
      if arg.is_a?(Prism::StringNode)
        how = node.name.to_s
        edges << { from: rel, to: point(base, rel, arg.unescaped, how),
                   at: "#{rel}:#{line}", how: how }
      else
        escapes << { in: rel, at: "#{rel}:#{line}",
                     how: "図に現れない読み込み ── #{node.name} に文字列以外を渡している" }
      end
    end
    queue.concat(node.compact_child_nodes)
  end
end

puts JSON.generate({ edges: edges, escapes: escapes, undecided: undecided })
