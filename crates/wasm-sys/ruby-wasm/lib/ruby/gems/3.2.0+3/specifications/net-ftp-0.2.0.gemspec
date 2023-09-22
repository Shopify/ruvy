# -*- encoding: utf-8 -*-
# stub: net-ftp 0.2.0 ruby lib

Gem::Specification.new do |s|
  s.name = "net-ftp".freeze
  s.version = "0.2.0"

  s.required_rubygems_version = Gem::Requirement.new(">= 0".freeze) if s.respond_to? :required_rubygems_version=
  s.metadata = { "homepage_uri" => "https://github.com/ruby/net-ftp", "source_code_uri" => "https://github.com/ruby/net-ftp" } if s.respond_to? :metadata=
  s.require_paths = ["lib".freeze]
  s.authors = ["Shugo Maeda".freeze]
  s.date = "2022-11-12"
  s.description = "Support for the File Transfer Protocol.".freeze
  s.email = ["shugo@ruby-lang.org".freeze]
  s.homepage = "https://github.com/ruby/net-ftp".freeze
  s.licenses = ["Ruby".freeze, "BSD-2-Clause".freeze]
  s.required_ruby_version = Gem::Requirement.new(">= 2.6.0".freeze)
  s.rubygems_version = "3.4.0.dev".freeze
  s.summary = "Support for the File Transfer Protocol.".freeze

  s.installed_by_version = "3.4.0.dev" if s.respond_to? :installed_by_version

  s.specification_version = 4

  s.add_runtime_dependency(%q<net-protocol>.freeze, [">= 0"])
  s.add_runtime_dependency(%q<time>.freeze, [">= 0"])
end
