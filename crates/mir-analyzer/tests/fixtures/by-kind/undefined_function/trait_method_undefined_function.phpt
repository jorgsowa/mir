===description===
Trait method bodies should detect undefined function calls
===file===
<?php
trait Auditable {
    public function audit(): void {
        nonexistent_function();
    }
}
===expect===
UndefinedFunction@4:9-4:31: Function nonexistent_function() is not defined
