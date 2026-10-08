===description===
The refinement from a literal-key write on a shape-typed property does not
survive conditional writes, loop writes, unset of the key, or an unrelated
read without a prior write.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    /** @var array{id: int, label?: string} */
    private array $data = ['id' => 1];

    public function noWrite(): string {
        return $this->data['label'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function conditionalWrite(bool $c): string {
        if ($c) {
            $this->data['label'] = 'x';
        }
        return $this->data['label'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    /** @param list<int> $xs */
    public function loopWrite(array $xs): string {
        foreach ($xs as $_) {
            $this->data['label'] = 'x';
        }
        return $this->data['label'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function writeThenUnset(): string {
        $this->data['label'] = 'x';
        unset($this->data['label']);
        return $this->data['label'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }
}
