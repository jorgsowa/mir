===description===
The framework validates the client IP it returns with `FILTER_VALIDATE_IP`.
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
    function clientIp(Symfony\Component\HttpFoundation\Request $request, Illuminate\Http\Request $laravel): void {
        echo $request->getClientIp();
        echo $laravel->ip();
    }
}
