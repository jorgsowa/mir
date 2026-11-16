===description===
A session ID taken from a query or form parameter through Symfony's `Request`
or a Laravel request allows session fixation; one read from the cookie, where
PHP itself reads it, doesn't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class InputBag {
        public function get(string $key): ?string { return null; }
    }
    class Request {
        public InputBag $query;
        public InputBag $cookies;
        public function __construct() {
            $this->query = new InputBag();
            $this->cookies = new InputBag();
        }
    }
}

namespace App {
    use Symfony\Component\HttpFoundation\Request;

    function resume(Request $request): void {
        session_id((string) $request->query->get('sid'));
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
        $fromCookie = (string) $request->cookies->get('PHPSESSID');
        session_id($fromCookie);
        $sid = (string) $request->query->get('sid');
        session_id($sid);
//      ^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
    }
}
