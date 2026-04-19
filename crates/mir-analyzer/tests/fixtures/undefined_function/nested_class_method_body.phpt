===source===
<?php
function outer(): void {
    class Inner {
        public function f(): void {
            nonexistent_function();
        }
    }
}
===expect===
UndefinedFunction: Function nonexistent_function() is not defined
