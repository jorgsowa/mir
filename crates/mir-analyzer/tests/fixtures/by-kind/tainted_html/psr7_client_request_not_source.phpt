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
        echo $request->getRequestTarget();
        echo $request->getHeaderLine('X-Name');
        echo $request->getHeader('X-Name')[0];
        echo $request->getHeaders()['X-Name'][0];
        echo $request->getBody()->getContents();
        echo $request->getQueryParams()['q'];
        echo $request->getMethod();
    }

    function serverImplementations(ServerRequest $a, AppRequest $b, WrappedServerRequest $c): void {
        echo $a->getHeaderLine('X-Name');
        echo $b->getBody()->getContents();
        echo $c->getUri();
    }

    function narrowedToServer(RequestInterface $request): void {
        if ($request instanceof ServerRequestInterface) {
            echo $request->getHeaderLine('X-Name');
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
    }
}
===expect===
TaintedHtml@56:8-56:32: Tainted HTML output — possible XSS
TaintedHtml@57:8-57:42: Tainted HTML output — possible XSS
TaintedHtml@58:8-58:47: Tainted HTML output — possible XSS
TaintedHtml@59:8-59:46: Tainted HTML output — possible XSS
TaintedHtml@60:8-60:49: Tainted HTML output — possible XSS
TaintedHtml@61:8-61:48: Tainted HTML output — possible XSS
TaintedHtml@62:8-62:45: Tainted HTML output — possible XSS
TaintedHtml@67:8-67:41: Tainted HTML output — possible XSS
TaintedHtml@68:8-68:42: Tainted HTML output — possible XSS
TaintedHtml@69:8-69:26: Tainted HTML output — possible XSS
TaintedHtml@74:12-74:51: Tainted HTML output — possible XSS
TaintedHtml@89:8-89:35: Tainted HTML output — possible XSS
