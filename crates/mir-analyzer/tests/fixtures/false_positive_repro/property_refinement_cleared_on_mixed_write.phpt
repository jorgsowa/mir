===description===
FP-B edge case: writing a mixed value to a narrowed property must clear the
prior guard-based refinement. Without the fix, the stale `string` refinement
(from the null-check guard) survives the `$this->data = mixed_fn()` assignment
and incorrectly silences a subsequent NullableReturnStatement.
===config===
php_version=8.2
===file===
<?php

/** @return mixed */
function mixed_fn(): mixed { return rand() ? "ok" : null; }

class Buffer {
    public ?string $data = null;

    public function load(): void {
        if ($this->data === null) {
            $this->data = 'initial';
        }
        // Overwrite with a mixed value — the prior `string` refinement must vanish.
        $this->data = mixed_fn();
    }

    public function get(): string {
        // After the mixed write, $this->data is ?string again (declared type),
        // so returning it directly without a null-check should emit NullableReturnStatement.
        return $this->data;
    }
}
===expect===
NullableReturnStatement@20:8-20:27: Return type 'string|null' is not compatible with declared 'string'
