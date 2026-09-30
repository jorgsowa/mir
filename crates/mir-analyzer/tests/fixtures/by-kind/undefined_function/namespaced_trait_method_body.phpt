===description===
namespaced trait method body
===file===
<?php
namespace App {
    trait MyTrait {
        public function go(): void {
            missing_function();
//          ^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function missing_function() is not defined
        }
    }
}
===expect===
