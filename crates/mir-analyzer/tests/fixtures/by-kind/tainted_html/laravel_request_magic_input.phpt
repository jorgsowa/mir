===description===
Laravel's `$request->name` and `$request['name']` read raw input. A declared property isn't magic input, and other array objects aren't requests.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class Request {
        public string $locale = 'en';
        public function getClientIp(): ?string { return null; }
    }
}

namespace Illuminate\Http {
    class Request extends \Symfony\Component\HttpFoundation\Request implements \ArrayAccess {
        public function __get(string $key): mixed { return null; }
        public function ip(): ?string { return null; }
        public function offsetExists(mixed $offset): bool { return false; }
        public function offsetGet(mixed $offset): mixed { return null; }
        public function offsetSet(mixed $offset, mixed $value): void {}
        public function offsetUnset(mixed $offset): void {}
    }
}

namespace {
    function magic(Illuminate\Http\Request $request): void {
        echo $request->name;
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request['name'];
//      ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $request->locale;
    }

    function otherClass(ArrayObject $a): void {
        echo $a['name'];
    }
}
