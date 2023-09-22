# -*- encoding: utf-8 -*-
# stub: syntax_suggest 0.0.1 ruby lib

Gem::Specification.new do |s|
  s.name = "syntax_suggest".freeze
  s.version = "0.0.1"

  s.required_rubygems_version = Gem::Requirement.new(">= 0".freeze) if s.respond_to? :required_rubygems_version=
  s.metadata = { "homepage_uri" => "https://github.com/ruby/syntax_suggest.git", "source_code_uri" => "https://github.com/ruby/syntax_suggest.git" } if s.respond_to? :metadata=
  s.require_paths = ["lib".freeze]
  s.authors = ["schneems".freeze]
  s.bindir = "exe".freeze
  s.date = "2022-11-12"
  s.description = "When you get an \"unexpected end\" in your syntax this gem helps you find it".freeze
  s.email = ["richard.schneeman+foo@gmail.com".freeze]
  s.files = ["lib/syntax_suggest.rb".freeze, "lib/syntax_suggest/api.rb".freeze, "lib/syntax_suggest/around_block_scan.rb".freeze, "lib/syntax_suggest/block_expand.rb".freeze, "lib/syntax_suggest/capture_code_context.rb".freeze, "lib/syntax_suggest/clean_document.rb".freeze, "lib/syntax_suggest/cli.rb".freeze, "lib/syntax_suggest/code_block.rb".freeze, "lib/syntax_suggest/code_frontier.rb".freeze, "lib/syntax_suggest/code_line.rb".freeze, "lib/syntax_suggest/code_search.rb".freeze, "lib/syntax_suggest/core_ext.rb".freeze, "lib/syntax_suggest/display_code_with_line_numbers.rb".freeze, "lib/syntax_suggest/display_invalid_blocks.rb".freeze, "lib/syntax_suggest/explain_syntax.rb".freeze, "lib/syntax_suggest/left_right_lex_count.rb".freeze, "lib/syntax_suggest/lex_all.rb".freeze, "lib/syntax_suggest/lex_value.rb".freeze, "lib/syntax_suggest/parse_blocks_from_indent_line.rb".freeze, "lib/syntax_suggest/pathname_from_message.rb".freeze, "lib/syntax_suggest/priority_engulf_queue.rb".freeze, "lib/syntax_suggest/priority_queue.rb".freeze, "lib/syntax_suggest/ripper_errors.rb".freeze, "lib/syntax_suggest/unvisited_lines.rb".freeze, "lib/syntax_suggest/version.rb".freeze]
  s.homepage = "https://github.com/ruby/syntax_suggest.git".freeze
  s.licenses = ["MIT".freeze]
  s.required_ruby_version = Gem::Requirement.new(">= 2.5.0".freeze)
  s.rubygems_version = "3.4.0.dev".freeze
  s.summary = "Find syntax errors in your source in a snap".freeze
end
