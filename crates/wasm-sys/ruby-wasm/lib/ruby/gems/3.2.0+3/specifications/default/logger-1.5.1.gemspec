# -*- encoding: utf-8 -*-
# stub: logger 1.5.1 ruby lib

Gem::Specification.new do |s|
  s.name = "logger".freeze
  s.version = "1.5.1"

  s.required_rubygems_version = Gem::Requirement.new(">= 0".freeze) if s.respond_to? :required_rubygems_version=
  s.require_paths = ["lib".freeze]
  s.authors = ["Naotoshi Seo".freeze, "SHIBATA Hiroshi".freeze]
  s.date = "2022-11-12"
  s.description = "Provides a simple logging utility for outputting messages.".freeze
  s.email = ["sonots@gmail.com".freeze, "hsbt@ruby-lang.org".freeze]
  s.files = ["lib/logger.rb".freeze, "lib/logger/errors.rb".freeze, "lib/logger/formatter.rb".freeze, "lib/logger/log_device.rb".freeze, "lib/logger/period.rb".freeze, "lib/logger/severity.rb".freeze, "lib/logger/version.rb".freeze]
  s.homepage = "https://github.com/ruby/logger".freeze
  s.licenses = ["Ruby".freeze, "BSD-2-Clause".freeze]
  s.required_ruby_version = Gem::Requirement.new(">= 2.3.0".freeze)
  s.rubygems_version = "3.4.0.dev".freeze
  s.summary = "Provides a simple logging utility for outputting messages.".freeze

  s.specification_version = 4

  s.add_development_dependency(%q<bundler>.freeze, [">= 0"])
  s.add_development_dependency(%q<rake>.freeze, [">= 12.3.3"])
  s.add_development_dependency(%q<test-unit>.freeze, [">= 0"])
  s.add_development_dependency(%q<rdoc>.freeze, [">= 0"])
end
