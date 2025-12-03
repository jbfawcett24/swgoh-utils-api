-- Create tables -- 

CREATE TABLE IF NOT EXISTS plan (
    id INTEGER PRIMARY KEY,
    planName TEXT NOT NULL UNIQUE,
    icon TEXT NOT NULL,
    allyCode TEXT NOT NULL,
    FOREIGN KEY (allyCode) REFERENCES account(allyCode)
);

CREATE TABLE IF NOT EXISTS charPlan (
    id INTEGER PRIMARY KEY,
    charName TEXT NOT NULL,
    goalStars INT NOT NULL,
    goalGear INT NOT NULL,
    goalRelic INT NOT NULL,
    baseId TEXT NOT NULL,
    planId INTEGER NOT NULL,
    FOREIGN KEY (baseId) REFERENCES rosterUnit(definitionId),
    FOREIGN KEY (planId) REFERENCES plan(id)
);

-- Select the plans --

SELECT planName, icon
FROM plan
WHERE allyCode = 'USER ALLY CODE';

SELECT charPlan.charName, unit.iconPath, charPlan.goalStars, charPlan.goalGear, charPlan.goalRelic, charPlan.baseId
FROM charPlan INNER JOIN plan ON charPlan.planId = plan.id
    INNER JOIN unit ON unit.baseId = charPlan.baseId
WHERE plan.allyCode = 'USER ALLY CODE';

-- INSERT INTO THE PLAN --

INSERT INTO plan
(planName, icon, allyCode)
VALUES (?, ?, ?);

INSERT INTO charPlan
(charName, goalStars, goalGear, goalRelic, baseId, planId)
VALUES (?, ?, ?, ?, ?, ?)