
puts "start of example ruby script"

class Discount
  attr_reader :discount
  def initialize(discount)
    @discount = discount
  end

  def puts_discount
    puts @discount
  end
end

output = {
  discount: Discount.new("a discount").discount,
  value: 100.0
}

custom_print(Inspector.inspect(output))