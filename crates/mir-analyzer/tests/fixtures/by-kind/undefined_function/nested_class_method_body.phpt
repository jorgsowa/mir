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
UndefinedFunction@5:13-5:35: Function nonexistent_function() is not defined
