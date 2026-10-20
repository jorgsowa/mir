===description===
A namespaced user function that shares its name with a builtin sink is not that
sink: `namespace App; array_map($_GET['l'], [1])` calls the user function. The
global builtin still fires when called with a leading `\`, from the global
namespace, or from a namespace that doesn't define its own function.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App {
    function array_map(string $label, array $items): array { return $items; }
    function call_user_func(string $label): void {}
    function usort(string $label, array $items): void {}

    function shadowed(): void {
        array_map($_GET['l'], [1]);
        call_user_func($_GET['l']);
        $items = [3, 1];
        usort($_GET['l'], $items);
    }

    function explicit_global(): void {
        \array_map($_GET['l'], [1]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
        \call_user_func($_GET['l']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}

namespace Other {
    function untouched(): void {
        array_map($_GET['l'], [1]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
        call_user_func($_GET['l']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}

namespace Imported {
    use function App\array_map;

    function through_import(): void {
        array_map($_GET['l'], [1]);
        \array_map($_GET['l'], [1]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}

namespace {
    function global_scope(): void {
        array_map($_GET['l'], [1]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}
