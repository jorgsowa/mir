===description===
A token read through a framework's request object and compared with `===` to a
stored secret leaks it through timing like `$_POST` does.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <ArgumentTypeCoercion errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class HeaderBag {
        public function get(string $key): ?string { return null; }
    }
    class Request {
        public HeaderBag $headers;
        public function __construct() { $this->headers = new HeaderBag(); }
    }
}

namespace App {
    use Symfony\Component\HttpFoundation\Request;

    final class Webhook {
        private string $secret = '';

        public function accept(Request $request): bool {
            return $request->headers->get('X-Secret') === $this->secret;
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TimingUnsafeComparison: the stored secret 'secret' is compared with ===, which returns sooner the fewer leading characters match; use hash_equals()
        }
    }
}
