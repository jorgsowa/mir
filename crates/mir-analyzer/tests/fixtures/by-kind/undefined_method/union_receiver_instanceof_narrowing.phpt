===description===
`instanceof` narrows a union receiver; the method exists on the narrowed member (mir even picks the wrong member).
===config===
php_version=8.4
===file===
<?php
class Holders { public function findFirst(): int { return 0; } }
class Relations { public function getFor(): int { return 0; } }
class Loader {
    private Holders|Relations $data;
    public function __construct(Holders|Relations $d) { $this->data = $d; }
    public function run(): int {
        // expect: UndefinedMethod (union member method after instanceof narrowing)
        if ($this->data instanceof Relations) {
            return $this->data->getFor();
        }
        return $this->data->findFirst();
    }
}
===expect===
