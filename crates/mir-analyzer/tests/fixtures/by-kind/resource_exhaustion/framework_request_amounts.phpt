===description===
Counts read through Symfony's `Request` and its parameter bags, a Laravel
request or a PSR-7 server request are request data like `$_GET`; a capped
value isn't reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <ArgumentTypeCoercion errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class InputBag {
        public function getInt(string $key, int $default = 0): int { return $default; }
    }
    class Request {
        public InputBag $query;
        public function __construct() { $this->query = new InputBag(); }
        public function get(string $key): mixed { return null; }
    }
}

namespace Psr\Http\Message {
    interface ServerRequestInterface {
        /** @return array<string, mixed> */
        public function getQueryParams(): array;
    }
}

namespace App {
    use Psr\Http\Message\ServerRequestInterface;
    use Symfony\Component\HttpFoundation\Request;

    function pad(Request $request, ServerRequestInterface $psr): void {
        echo str_repeat('-', $request->query->getInt('width'));
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: str_repeat() takes its size, count or duration straight from the request — possible denial of service
        $count = (int) $request->get('count');
        range(1, $count);
//      ^^^^^^^^^^^^^^^^ ResourceExhaustion: range() takes its size, count or duration straight from the request — possible denial of service
        sleep((int) $psr->getQueryParams()['delay']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: sleep() takes its size, count or duration straight from the request — possible denial of service
        $rows = $request->query->getInt('rows');
        if ($rows > 100) {
            return;
        }
        array_fill(0, $rows, null);
    }
}
