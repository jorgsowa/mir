===file===
<?php
class Foo
{
    protected string $var = '';

    /**
     * @return ($var is null ? string : $this)
     */
    public function bar(?string $var = null): static|string
    {
        if ($var === null) {
            return $this->var;
        }
        $this->var = $var;
        return $this;
    }
}
===expect===
