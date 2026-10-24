===description===
Only a PSR-7 `ServerRequestInterface` carries incoming client data. A plain
`RequestInterface` is an outgoing request the caller built, so its URI, headers
and body are not sources, while implementations and sub-interfaces of the server
request stay tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <InvalidReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Psr\Http\Message {
    interface StreamInterface {
        public function getContents(): string;
    }
    interface UriInterface {
        public function __toString(): string;
    }
    interface RequestInterface {
        public function getMethod(): string;
        public function getUri(): UriInterface;
        public function getRequestTarget(): string;
        public function getHeader(string $name): array;
        public function getHeaders(): array;
        public function getHeaderLine(string $name): string;
        public function getBody(): StreamInterface;
        public function withHeader(string $name, string $value): RequestInterface;
    }
    interface ServerRequestInterface extends RequestInterface {
        public function getQueryParams(): array;
    }
}

namespace App {
    use Psr\Http\Message\RequestInterface;
    use Psr\Http\Message\ServerRequestInterface;

    interface AppRequest extends ServerRequestInterface {}
    abstract class ServerRequest implements ServerRequestInterface {}
    abstract class ClientRequest implements RequestInterface {}
    abstract class WrappedServerRequest extends ServerRequest {}

    function client(RequestInterface $request): void {
        echo $request->getMethod() . ' ' . $request->getUri();
        echo $request->getRequestTarget();
        echo $request->getHeaderLine('X-Name');
        echo $request->getHeader('X-Name')[0];
        echo $request->getHeaders()['X-Name'][0];
        echo $request->getBody()->getContents();
        echo (string) $request->getBody();
        echo $request->withHeader('X-A', 'b')->getHeaderLine('X-A');
    }

    function clientImplementation(ClientRequest $request): void {
        echo $request->getHeaderLine('X-Name');
        echo $request->getBody()->getContents();
    }

    function nullableClient(?RequestInterface $request): void {
        if ($request !== null) {
            echo $request->getUri();
        }
    }

    function server(ServerRequestInterface $request): void {
        echo $request->getUri();
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getRequestTarget();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getHeaderLine('X-Name');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getHeader('X-Name')[0];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getHeaders()['X-Name'][0];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getBody()->getContents();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getQueryParams()['q'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->getMethod();
    }

    function serverImplementations(ServerRequest $a, AppRequest $b, WrappedServerRequest $c): void {
        echo $a->getHeaderLine('X-Name');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $b->getBody()->getContents();
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $c->getUri();
//      ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function narrowedToServer(RequestInterface $request): void {
        if ($request instanceof ServerRequestInterface) {
            echo $request->getHeaderLine('X-Name');
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
        echo $request->getHeaderLine('X-Name');
    }

    function clientTarget(RequestInterface $request): string {
        return $request->getRequestTarget();
    }

    function serverTarget(ServerRequestInterface $request): string {
        return $request->getRequestTarget();
    }

    function throughHelpers(RequestInterface $client, ServerRequestInterface $server): void {
        echo clientTarget($client);
        echo serverTarget($server);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
