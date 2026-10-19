===description===
Only the `$request->server` entries a client controls are tainted, the same
keys as `$_SERVER`. Server-set entries, and keys that can't name a request entry,
are clean. A default argument and a tainted key still carry their own taint,
and reading the whole bag stays tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <PropertyNotSetInConstructor errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <InvalidReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class ParameterBag {
        public function get(string $key, mixed $default = null): mixed { return $default; }
        public function getString(string $key, string $default = ''): string { return $default; }
        public function all(): array { return []; }
        public function getInt(string $key): int { return 0; }
    }
    class InputBag extends ParameterBag {}
    class ServerBag extends ParameterBag {
        public function getHeaders(): array { return []; }
    }
    class Request {
        public InputBag $query;
        public ServerBag $server;
        public function getContent(): string { return ''; }
    }
}

namespace Illuminate\Http {
    class Request extends \Symfony\Component\HttpFoundation\Request {}
}

namespace App {
    use Symfony\Component\HttpFoundation\Request;

    function serverControlled(Request $request): void {
        echo $request->server->get('DOCUMENT_ROOT');
        echo $request->server->get('SCRIPT_NAME');
        echo $request->server->get('REMOTE_ADDR');
        echo $request->server->get('APP_ENV');
        echo $request->server->getString('SERVER_SOFTWARE');
        echo $request->server->get('DOCUMENT_ROOT', 'fallback');
        echo $request?->server->get('DOCUMENT_ROOT');
        echo ($request->server)->get('DOCUMENT_ROOT');
        echo $request->server->getInt('REQUEST_TIME');
    }

    function requestControlled(Request $request, string $header): void {
        echo $request->server->get('HTTP_USER_AGENT');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get('REQUEST_URI');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get('QUERY_STRING');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->getString('PHP_AUTH_USER');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get("HTTP_$header");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get('HTTP_' . $header);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request?->server->get('HTTP_HOST');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->all()['HTTP_HOST'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->getHeaders()['HOST'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get('DOCUMENT_ROOT', $_GET['d']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->server->get($_GET['k']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->query->get('q');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function dynamicKeys(Request $request, string $name): void {
        echo $request->server->get($name);
        $key = 'DOCUMENT_ROOT';
        echo $request->server->get($key);
        $key = 'HTTP_HOST';
        echo $request->server->get($key);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function laravel(\Illuminate\Http\Request $request): void {
        echo $request->server->get('DOCUMENT_ROOT');
        echo $request->server->get('HTTP_REFERER');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    class Controller {
        public function __construct(private Request $request) {}

        private function env(string $name): string {
            return (string) $this->request->server->get($name);
        }

        private function documentRoot(): string {
            return (string) $this->request->server->get('DOCUMENT_ROOT');
        }

        private function userAgent(): string {
            return (string) $this->request->server->get('HTTP_USER_AGENT');
        }

        private function fromDefault(string $fallback): string {
            return (string) $this->request->server->get('DOCUMENT_ROOT', $fallback);
        }

        public function show(): void {
            echo $this->env('HOME');
            echo $this->documentRoot();
            echo $this->fromDefault('x');
            echo $this->userAgent();
//          ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
            echo $this->env($_GET['k']);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
            echo $this->fromDefault($_GET['d']);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }
}
===expect===
TaintedHtml@42:8-42:50: Tainted HTML output — possible XSS
TaintedHtml@44:8-44:51: Tainted HTML output — possible XSS
TaintedHtml@48:8-48:51: Tainted HTML output — possible XSS
TaintedHtml@50:8-50:54: Tainted HTML output — possible XSS
TaintedHtml@52:8-52:49: Tainted HTML output — possible XSS
TaintedHtml@54:8-54:50: Tainted HTML output — possible XSS
TaintedHtml@56:8-56:52: Tainted HTML output — possible XSS
TaintedHtml@58:8-58:64: Tainted HTML output — possible XSS
TaintedHtml@104:12-104:36: Tainted HTML output — possible XSS
