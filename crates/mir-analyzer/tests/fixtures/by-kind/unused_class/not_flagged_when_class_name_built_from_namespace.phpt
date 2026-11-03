===description===
A class name built from a namespace literal (`'App\Components\\' . $name`) may name any class in that namespace
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
namespace App\Components;

final class Alert {}
final class Confirm {}

namespace App;

final class Unrelated {}
//    ^^^^^^^^^^^^^^^^^^ UnusedClass: Class App\Unrelated is never referenced

final class Factory {
    public function make(string $method): object {
        $component = '\App\Components\\' . ucfirst($method);
        return new $component();
//                 ^^^^^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'non-empty-string'
    }
}

(new Factory())->make('alert');
