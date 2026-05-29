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
UndefinedFunction@5:13-5:31: Function missing_function() is not defined
