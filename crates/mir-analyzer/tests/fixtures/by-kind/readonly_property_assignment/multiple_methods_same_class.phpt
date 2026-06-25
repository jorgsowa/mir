===description===
Multiple methods in declaring class can each init different readonly properties
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class ValueObject {
    public readonly int $id;
    public readonly string $name;

    public function setId(int $id): void {
        $this->id = $id;
    }

    public function setName(string $name): void {
        $this->name = $name;
    }
}
===expect===
