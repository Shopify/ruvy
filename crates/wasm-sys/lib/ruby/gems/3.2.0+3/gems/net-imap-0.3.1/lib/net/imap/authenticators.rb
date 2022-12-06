# frozen_string_literal: true

# Registry for SASL authenticators used by Net::IMAP.
module Net::IMAP::Authenticators

  # Adds an authenticator for use with Net::IMAP#authenticate.  +auth_type+ is the
  # {SASL mechanism}[https://www.iana.org/assignments/sasl-mechanisms/sasl-mechanisms.xhtml]
  # supported by +authenticator+ (for instance, "+PLAIN+").  The +authenticator+
  # is an object which defines a +#process+ method to handle authentication with
  # the server.  See Net::IMAP::PlainAuthenticator, Net::IMAP::LoginAuthenticator,
  # Net::IMAP::CramMD5Authenticator, and Net::IMAP::DigestMD5Authenticator for
  # examples.
  #
  # If +auth_type+ refers to an existing authenticator, it will be
  # replaced by the new one.
  def add_authenticator(auth_type, authenticator)
    authenticators[auth_type] = authenticator
  end

  # Builds an authenticator for Net::IMAP#authenticate.  +args+ will be passed
  # directly to the chosen authenticator's +#initialize+.
  def authenticator(mechanism, *authargs, **properties, &callback)
    authenticator = authenticators.fetch(mechanism.upcase) do
      raise ArgumentError, 'unknown auth type - "%s"' % mechanism
    end
    if authenticator.respond_to?(:new)
      authenticator.new(*authargs, **properties, &callback)
    else
      authenticator.call(*authargs, **properties, &callback)
    end
  end

  private

  def authenticators
    @authenticators ||= {}
  end

end

Net::IMAP.extend Net::IMAP::Authenticators

require_relative "authenticators/plain"

require_relative "authenticators/login"
require_relative "authenticators/cram_md5"
require_relative "authenticators/digest_md5"
require_relative "authenticators/xoauth2"
