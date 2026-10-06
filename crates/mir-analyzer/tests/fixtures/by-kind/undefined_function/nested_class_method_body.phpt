===description===
nested class method body
===file===
<?php
function outer(): void {
    class Inner {
        public function f(): void {
            nonexistent_function();
//          ^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function nonexistent_function() is not defined
        }
    }
}
