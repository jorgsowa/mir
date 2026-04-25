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
UndefinedFunction: Function missing_function() is not defined
