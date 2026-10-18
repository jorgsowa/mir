===description===
A class name held in a string is always fully qualified at runtime: neither
the current namespace nor `use` imports apply to it. `'Exception'` names the
global class, `'Thing'` stays `Thing` even with `use Lib\Thing`, and strings
that can't be class names (`'0'`, `''`) are never looked up.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    final class Thing {}
}

namespace App {
    use Lib\Thing;

    final class Factory {
        /**
         * @template T of object
         * @param class-string<T> $class
         */
        public static function make(string $class): void {}

        /** @param interface-string $iface */
        public static function contract(string $iface): void {}

        public static function call(callable $fn): void {}
    }

    function run(string $maybe): void {
        Factory::make('Exception');
        Factory::make('\Exception');
        Factory::make('Lib\Thing');
        Factory::make('Thing');
        Factory::make('Missing');
        Factory::contract('Countable');
        Factory::call('Exception::getMessage');
        if (!$maybe) {
            Factory::make($maybe);
        }
    }
}
===expect===
UndefinedClass@26:22-26:29: Class Thing does not exist
UndefinedClass@27:22-27:31: Class Missing does not exist
