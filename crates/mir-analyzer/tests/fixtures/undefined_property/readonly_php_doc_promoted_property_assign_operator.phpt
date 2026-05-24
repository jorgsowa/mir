===description===
readonlyPhpDocPromotedPropertyAssignOperator
===file===
<?php

final class A
{
    public function __construct(
        /**
         * @psalm-readonly
         */
        private string $string,
    ) {
    }

    private function mutateString(): void
    {
        $this->string = "";
    }
}
===expect===
InaccessibleProperty
===ignore===
TODO
