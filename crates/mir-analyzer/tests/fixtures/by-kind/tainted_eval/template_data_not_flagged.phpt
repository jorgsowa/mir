===description===
Request data passed to a template as variables, a fixed template with request data
in its context, and a Smarty file resource aren't code injection.
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
    }

    function render(Twig\Environment $twig, Smarty $smarty): void {
        $twig->createTemplate('Hello {{ name }}');
        Illuminate\Support\Facades\Blade::render('Hello {{ $name }}', ['name' => $_GET['name']]);
        $smarty->fetch('file:' . basename((string) $_GET['page']) . '.tpl');
    }
}
