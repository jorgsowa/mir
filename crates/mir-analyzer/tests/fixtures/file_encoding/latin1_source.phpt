===description===
A file containing non-UTF-8 (Latin-1) bytes is valid PHP: it is analyzed, and its declarations resolve from other files
===file:ValueWrapper.php===
<?php
class ValueWrapper {
    const LABEL = 'café';
    public function value(): string { return 'déjà vu'; }
    public function broken(): int {
        return $this->value();
//      ^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'string' is not compatible with declared 'int'
    }
}
===file:use.php===
<?php
function use_wrapper(): int {
    return (new ValueWrapper())->value();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'string' is not compatible with declared 'int'
}
