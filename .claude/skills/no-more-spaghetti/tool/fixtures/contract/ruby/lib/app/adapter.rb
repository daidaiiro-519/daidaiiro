require "app/core"
require "app/stray"
require "app/gen/made"
require "app/#{name}"
if fast
  require "app/core/fast"
else
  require "app/core/slow"
end
