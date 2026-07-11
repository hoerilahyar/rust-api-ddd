-- Extends the generic trigger_set_updated_at() function (see migration 000016)
-- to the new menus table.
CREATE TRIGGER set_updated_at_menus
    BEFORE UPDATE ON menus
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();
