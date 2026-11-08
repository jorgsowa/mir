===description===
A top-level `class_alias()` with literal names defines the alias as the original class.
===file===
<?php
namespace Orig {
    abstract class Yaml {
        public static function decode(): array { return []; }
        abstract public function m(): void;
    }
    class_alias('\Orig\Yaml', 'Alias\Yaml');
}

namespace App {
    use Alias\Yaml;
    function f(): array { return Yaml::decode(); }
}
