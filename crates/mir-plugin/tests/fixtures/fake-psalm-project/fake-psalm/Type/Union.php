<?php
namespace Psalm\Type;

final class Union
{
    public function __construct(private string $repr)
    {
    }

    public function __toString(): string
    {
        return $this->repr;
    }
}
