===description===
nested class method body
===file===
<?php
function outer(): void {
    class Inner {
        public function f(): void {
            nonexistent_function();
        }
    }
}
===expect===
UndefinedFunction@5:12-5:34: Function nonexistent_function() is not defined
