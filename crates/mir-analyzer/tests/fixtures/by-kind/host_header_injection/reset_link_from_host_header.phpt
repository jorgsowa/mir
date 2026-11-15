===description===
A link in an email built from the `Host` header points the victim at whatever host
the attacker sent.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class Request {
        public function getSchemeAndHttpHost(): string { return ''; }
    }
}

namespace App {
    use Symfony\Component\HttpFoundation\Request;

    function sendReset(string $email, string $token): void {
        $link = 'https://' . $_SERVER['HTTP_HOST'] . '/reset?token=' . $token;
        mail($email, 'Password reset', "Reset your password: $link");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HostHeaderInjection: Email body built from the request's Host header — possible password-reset link poisoning
    }

    function sendConfirm(Request $request, string $email): void {
        $base = $request->getSchemeAndHttpHost();
        $body = sprintf('Confirm: %s/confirm', $base);
        mail($email, 'Confirm', $body);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HostHeaderInjection: Email body built from the request's Host header — possible password-reset link poisoning
    }

    function forwarded(string $email): void {
        mail($email, 'Hi', 'Visit https://' . ($_SERVER['HTTP_X_FORWARDED_HOST'] ?? 'example.com'));
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HostHeaderInjection: Email body built from the request's Host header — possible password-reset link poisoning
    }
}
