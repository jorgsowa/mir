===description===
A guard on one nullable field does not narrow a separate nullable field that is returned.
===file===
<?php
final class Property {
    private ?int $id = null;
    private ?int $propertyId = null;

    public function isCreated(): bool {
        return $this->id !== null;
    }

    public function propertyId(): int {
        if (!$this->isCreated()) {
            throw new \LogicException();
        }
        /** @mir-check $this->propertyId is int|null */
        return $this->propertyId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }

    public function propertyIdDirectGuard(): int {
        if ($this->id === null) {
            throw new \LogicException();
        }
        return $this->propertyId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }

    public function propertyIdGuarded(): int {
        if ($this->propertyId === null) {
            throw new \LogicException();
        }
        /** @mir-check $this->propertyId is int */
        return $this->propertyId;
    }

    public function idGuarded(): int {
        if (!$this->isCreated()) {
            throw new \LogicException();
        }
        /** @mir-check $this->id is int */
        return $this->id;
    }
}
