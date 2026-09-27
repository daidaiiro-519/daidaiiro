require "minitest/autorun"
require_relative "../order"
def scenario(id) = (p = ENV["SCENARIO_TRACE"]) && File.open(p, "a") { |f| f.puts id }
class OrderTest < Minitest::Test
  [[[], "SC-01J7Q4M", false], [["りんご"], "SC-01J7Q4N", true]].each_with_index do |(lines, id, ok), i|
    define_method("test_確定_#{i}") do
      scenario(id)
      o = Order.new(lines)
      ok ? (o.confirm; assert_equal "確定済", o.status) : assert_raises(ArgumentError) { o.confirm }
    end
  end
end
