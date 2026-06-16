===description===
Absolute FQCN in docblock (leading backslash) is not mis-resolved via use aliases.
\Carbon\CarbonImmutable should not be confused with an Illuminate\Support\Carbon subpath.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class CarbonImmutable {}

class Carbon {
    public static function now(): CarbonImmutable {
        return new CarbonImmutable();
    }
}

class Foo {
    /**
     * @param \App\CarbonImmutable $dt
     */
    public function bar(\App\CarbonImmutable $dt): void {}
}

$foo = new Foo();
$foo->bar(Carbon::now());
===expect===
