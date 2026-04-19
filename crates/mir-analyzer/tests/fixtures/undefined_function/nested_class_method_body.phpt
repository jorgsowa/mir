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
UndefinedFunction: nonexistent_function()
