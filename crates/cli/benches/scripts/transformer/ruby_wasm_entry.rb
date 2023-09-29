require_relative "preload/transformer.rb"

input = STDIN.gets.strip
transformer = Transformer.new(input)
puts transformer.transform
