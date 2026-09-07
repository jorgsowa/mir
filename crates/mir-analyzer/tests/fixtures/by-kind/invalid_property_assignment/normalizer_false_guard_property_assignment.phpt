===description===
A string property assignment guarded against normalizer failure is valid.
===file===
<?php
class Text {
    public string $value = '';

    public function normalize(): void {
        $normalized = normalizer_normalize($this->value);
        if ($normalized !== false) {
            $this->value = $normalized;
        }
    }
}
===expect===
