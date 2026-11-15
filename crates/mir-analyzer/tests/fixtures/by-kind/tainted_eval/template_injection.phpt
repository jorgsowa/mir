===description===
Request data compiled as a template runs as code: Twig's `createTemplate()`,
Blade's `render()` and Smarty's `string:` and `eval:` resources.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Twig {
    class Environment {
        public function createTemplate(string $template): object { return new \stdClass(); }
    }
}

namespace Illuminate\Support\Facades {
    class Blade {
        public static function render(string $string, array $data = []): string { return ''; }
    }
}

namespace {
    class Smarty {
        public function fetch(string $template): string { return ''; }
        public function display(string $template): void {}
    }

    function twig(Twig\Environment $twig): void {
        $twig->createTemplate('Hello ' . $_GET['name']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
    }

    function blade(): void {
        Illuminate\Support\Facades\Blade::render($_POST['template']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
    }

    function smarty(Smarty $smarty): void {
        $smarty->fetch('string:' . $_GET['tpl']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
        $smarty->display("eval:{$_GET['tpl']}");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted code — possible code injection
    }
}
