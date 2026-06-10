===description===
Readonly php doc promoted property assign operator
===ignore===
TODO
===file===
<?php

final class A
{
    public function __construct(
        /**
         * @readonly
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
