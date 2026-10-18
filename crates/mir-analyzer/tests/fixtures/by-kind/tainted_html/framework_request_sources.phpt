===description===
Symfony, Laravel and PSR-7 request accessors return client data, and so do
the parameter bags of a Symfony request. A tainted object's methods and
properties carry its taint.
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
        public function all(): array { return []; }
        public function getInt(string $key): int { return 0; }
    }
    class InputBag extends ParameterBag {}
    class Request {
        public InputBag $query;
        public InputBag $request;
        public ParameterBag $attributes;
        public function get(string $key): mixed { return null; }
        public function getContent(): string { return ''; }
        public function getMethod(): string { return 'GET'; }
    }
}

namespace Illuminate\Http {
    class Request extends \Symfony\Component\HttpFoundation\Request {
        public function input(?string $key = null): mixed { return null; }
        public function boolean(string $key): bool { return false; }
    }
}

namespace Psr\Http\Message {
    interface StreamInterface {
        public function getContents(): string;
    }
    interface RequestInterface {
        public function getHeaderLine(string $name): string;
        public function getBody(): StreamInterface;
    }
    interface ServerRequestInterface extends RequestInterface {
        public function getQueryParams(): array;
        public function getParsedBody(): mixed;
    }
}

namespace {
    function request(?string $key = null): mixed { return null; }
    class Request {
        public static function input(string $key): mixed { return null; }
    }
}

namespace App {
    use Symfony\Component\HttpFoundation\Request;
    use Psr\Http\Message\ServerRequestInterface;

    function symfony(Request $request): void {
        echo $request->query->get('q');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->get('q');
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getContent();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        $bag = $request->request;
        echo $bag->all()['name'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->query->getInt('page');
        echo $request->getMethod();
    }

    function laravel(\Illuminate\Http\Request $request): void {
        echo $request->input('name');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->query->get('q');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo request('name');
//      ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo \Request::input('name');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->boolean('flag') ? 'yes' : 'no';
    }

    function psr(ServerRequestInterface $request): void {
        echo $request->getQueryParams()['q'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getHeaderLine('X-Name');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getBody()->getContents();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo (string) $request->getBody();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    class Controller {
        public function __construct(private Request $request) {}

        private function search(): string {
            return (string) $this->request->query->get('q');
        }

        public function show(): void {
            echo $this->search();
//          ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }

    function decoded(): void {
        $payload = json_decode((string) $_POST['payload']);
        echo $payload->name;
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $payload->author->name;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
