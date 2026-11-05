===description===
`HTMLPurifier::purify` and Symfony's filtering bag getters clear what they filter out; a plain bag getter doesn't.
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
    class ParameterBag {
        public function get(string $key): mixed { return null; }
        public function getDigits(string $key): string { return ''; }
        public function getAlnum(string $key): string { return ''; }
    }
    class InputBag extends ParameterBag {}
    class Request {
//  ^^^^^^^^^^^^^^^ MissingConstructor: Class Symfony\Component\HttpFoundation\Request has uninitialized properties but no constructor
        public InputBag $query;
    }
}

namespace {
    class HTMLPurifier {
        public function purify(string $html): string { return $html; }
    }

    function purified(HTMLPurifier $purifier): void {
        echo $purifier->purify($_GET['x']);
    }

    function bag(Symfony\Component\HttpFoundation\Request $request): void {
        echo $request->query->getDigits('a');
        echo $request->query->getAlnum('b');
        echo $request->query->get('c');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
