-- Add explanation to trades
ALTER TABLE trades ADD COLUMN reason TEXT;

-- Add Kelly metric to strategies (default safe value)
ALTER TABLE strategies ADD COLUMN kelly_fraction DOUBLE PRECISION DEFAULT 0.1;

-- Add execution mode to sessions ('sync' = immediate, 'edge' = wait for crossover)
ALTER TABLE sessions ADD COLUMN execution_mode TEXT NOT NULL DEFAULT 'sync';

-- Add portfolio allocation logic to sessions (how much of the total portfolio this session represents)
ALTER TABLE sessions ADD COLUMN allocated_weight DOUBLE PRECISION DEFAULT 1.0;
