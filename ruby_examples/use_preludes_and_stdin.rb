input = STDIN.gets.strip
class Discount
  attr_reader :input
  def initialize(input)
    @input = input
  end
end

output = {
  discount_input: Discount.new(input).input,
  value: 100.0
}

custom_print(Inspector.inspect(output))
$stdout.flush
