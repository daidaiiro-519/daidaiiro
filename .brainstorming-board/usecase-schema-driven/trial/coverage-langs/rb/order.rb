class Order
  attr_accessor :status, :lines
  def initialize(lines = [])
    @status = "準備中"
    @lines = lines
  end

  def confirm
    raise ArgumentError, "明細が0件" if @lines.empty?
    @status = "確定済"
  end
end
