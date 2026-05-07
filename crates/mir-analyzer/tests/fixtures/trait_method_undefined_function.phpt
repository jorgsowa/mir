--TEST--
Trait method bodies should detect undefined function calls
--FILE--
<?php
trait Auditable {
    public function audit(): void {
        nonexistent_function();
    }
}
--EXPECT--
undefined_function: nonexistent_function (line 4, col 8-30)
