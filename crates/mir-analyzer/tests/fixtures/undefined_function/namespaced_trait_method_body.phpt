===description===
namespaced trait method body
===file===
<?php
namespace App {
    trait MyTrait {
        public function go(): void {
            missing_function();
        }
    }
}
===expect===
UndefinedFunction@5:12: Function missing_function() is not defined
===ignore===
TODO
