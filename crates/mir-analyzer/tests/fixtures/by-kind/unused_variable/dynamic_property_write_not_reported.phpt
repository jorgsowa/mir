===description===
variable used as dynamic property name in write is not reported
===file===
<?php
class QueryBuilder {
    public int $offset = 0;
    public int $unionOffset = 0;

    public function offset(bool $useUnion, int $value): static {
        $property = $useUnion ? 'unionOffset' : 'offset';
        $this->$property = max(0, $value);
        return $this;
    }
}
===expect===
