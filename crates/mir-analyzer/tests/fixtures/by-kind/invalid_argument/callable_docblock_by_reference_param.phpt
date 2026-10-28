===description===
`array &$options` in a `Closure(...)` docblock is a by-reference `array` parameter, not a class named `array &`
===file===
<?php
namespace App;

final class Client {
    /** @param \Closure(string $host, array &$options): void $onRedirect */
    public function follow(\Closure $onRedirect): void {
        $options = [];
        $onRedirect('h', $options);
        $onRedirect('h', []);
//                       ^^ InvalidPassByReference: Argument $options of {closure}() must be passed by reference
    }

    /** @param callable(int &...$rest): void $f */
    public function each(callable $f): void {
        $a = 1;
        $f($a);
    }

    public function run(): void {
        $this->follow(static function (string $host, array &$options): void {
            $options[] = $host;
        });
    }
}
