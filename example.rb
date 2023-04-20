
# TODO: Should the user program read from STDIN? 
# or should we have a SHOPIFY_INPUT global defined and ready to use by user program?
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