===file===
<?php
trait Auditable {
    public function audit(): void {
        nonexistent_function();
    }
}
===expect===
UndefinedFunction: Function nonexistent_function() is not defined
