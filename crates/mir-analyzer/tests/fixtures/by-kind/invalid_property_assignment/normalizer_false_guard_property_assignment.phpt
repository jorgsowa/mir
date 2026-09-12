===description===
A normalizer failure guard permits string property assignment.
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
