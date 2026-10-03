use sim::SimWorld;
use sim::script::{Catalog, LevelData, NativeRegistry, Program, Value};
use std::collections::BTreeMap;

pub fn install(world: &mut SimWorld) -> Result<(), String> {
    let sources = BTreeMap::from([
        ("codescripts/struct".to_owned(), STRUCTS.to_owned()),
        ("survival/session".to_owned(), SESSION.to_owned()),
        (
            "maps/mp/gametypes/_callbacksetup".to_owned(),
            CALLBACKS.to_owned(),
        ),
    ]);
    let program = Program::load(
        &sources,
        &[
            "codescripts/struct",
            "survival/session",
            "maps/mp/gametypes/_callbacksetup",
        ],
        &Catalog::iw4(),
    )
    .map_err(|e| e.to_string())?;
    let level = LevelData {
        entities: vec![vec![("classname".to_owned(), "worldspawn".to_owned())]],
        ..Default::default()
    };
    world
        .install_gsc_program(program, NativeRegistry::default(), level)
        .map_err(|e| e.to_string())?;
    world
        .start_gsc("survival/session::main", Value::level(), Vec::new())
        .map_err(|e| e.to_string())?;
    Ok(())
}

const STRUCTS: &str = r#"
initstructs()
{
    level.struct = [];
    level.connectedplayers = 0;
    level.sessiondeaths = 0;
    level.sessionready = false;
}
"#;

const SESSION: &str = r#"
main()
{
    level.sessionname = "Authored survival world";
    level.sessionstartedat = gettime();
    maps\mp\gametypes\_callbacksetup::codecallback_startgametype();
}

respawn(client, origin, angles)
{
    players = getentarray("player", "classname");
    for (i = 0; i < players.size; i++)
    {
        player = players[i];
        if (player getentitynumber() == client && player.health <= 0)
        {
            player.sessionstate = "playing";
            player spawn(origin, angles);
            player giveweapon("authored_carbine");
            player switchtoweaponimmediate("authored_carbine");
        }
    }
}

restore(client, alive, origin, angles, velocity, health, clip, stock, kills, deaths, score)
{
    players = getentarray("player", "classname");
    for (i = 0; i < players.size; i++)
    {
        player = players[i];
        if (player getentitynumber() != client)
            continue;
        player.kills = kills;
        player.score = score;
        if (alive)
        {
            player.deaths = deaths;
            if (player.health <= 0)
            {
                player.sessionstate = "playing";
                player spawn(origin, angles);
                player giveweapon("authored_carbine");
                player switchtoweaponimmediate("authored_carbine");
            }
            player setorigin(origin);
            player setplayerangles(angles);
            player setvelocity(velocity);
            player.health = health;
            player setweaponammoclip("authored_carbine", clip);
            player setweaponammostock("authored_carbine", stock);
        }
        else if (player.health > 0)
        {
            player setorigin(origin);
            player setplayerangles(angles);
            player.deaths = deaths - 1;
            player suicide();
        }
        else
        {
            player.deaths = deaths;
        }
    }
}
"#;

const CALLBACKS: &str = r#"
codecallback_startgametype()
{
    level.sessionready = true;
    level notify("prematch_over");
}

codecallback_playerconnect()
{
    self.sessionteam = "none";
    if (self.health > 0)
        self.sessionstate = "playing";
    else
        self.sessionstate = "dead";
    self.kills = 0;
    self.deaths = 0;
    self.score = 0;
    self.pers["connected"] = true;
    level.connectedplayers++;
}

codecallback_playerdisconnect()
{
    self.pers["connected"] = false;
    if (level.connectedplayers > 0)
        level.connectedplayers--;
    self notify("disconnect");
}

codecallback_playerdamage(inflictor, attacker, amount, flags, means, weapon, point, direction, hitlocation, offset)
{
    if (self.health <= 0 || self.sessionstate != "playing" || amount <= 0)
        return;
    self finishplayerdamage(inflictor, attacker, amount, flags, means, weapon, point, direction, hitlocation, offset);
}

codecallback_playerkilled(inflictor, attacker, amount, means, weapon, direction, hitlocation, offset, animation)
{
    self.sessionstate = "dead";
    self.deaths++;
    self.pers["lastdeathat"] = gettime();
    level.sessiondeaths++;
    if (isdefined(attacker) && isplayer(attacker) && attacker != self)
    {
        attacker.kills++;
        attacker.score++;
    }
}

codecallback_playerlaststand(inflictor, attacker, amount, means, weapon, direction, hitlocation, offset, animation)
{
    self suicide();
}
"#;
