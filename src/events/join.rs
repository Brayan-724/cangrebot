use anyhow::Result;
use plantita_welcomes::create_welcome::combine_images;
use serenity::all::{CreateAttachment, CreateMessage};
use serenity::{model::prelude::*, prelude::*};
use std::convert::TryFrom;

const WELCOME_MESSAGE: &str = r#"¡Bienvenidx a la Comunidad de RustLangES!

Nos alegra que hayas decidido unirte a nuestra comunidad. Aquí encontrarás varios canales dedicados a diferentes aspectos de nuestra comunidad:

- [#anuncios-de-la-comunidad](<https://discord.com/channels/778674594856960012/1159719259287597087>): Este es el lugar donde compartimos las últimas novedades y eventos de nuestra comunidad. ¡Mantente al tanto de lo que está sucediendo!
- [#show-case](<https://discord.com/channels/778674594856960012/1144727580323369000>): ¿Has creado algo increíble con Rust? ¡Este es el canal perfecto para compartirlo con el resto de la comunidad!
- [#proyectos-comunitarios](<https://discord.com/channels/778674594856960012/1140802416170770463>): Aquí se discuten los proyectos que estamos desarrollando como comunidad, como nuestra página web, blog y bot. ¡Participa y ayúdanos a mejorar!
- [#retos-diarios](<https://discord.com/channels/778674594856960012/1219703076944871616>): ¿Quieres poner a prueba tus habilidades de programación? ¡Únete a los retos diarios y comparte tus soluciones!
- [#principiantes](<https://discord.com/channels/778674594856960012/795836875872141362>): Si estas empezando en Rust, este es el lugar perfecto para encontrar ayuda y recursos para empezar.

Recuerda revisar los mensajes fijados en cada canal para obtener más información.

> **Nota:** Es posible que para acceder a algunos canales necesites de un rol especifico
> por lo que te recomendamos que te asignes los roles que te interesen

¡No olvides seguirnos en nuestras redes sociales y visitar nuestras webs para mantenerte al día con todo lo que sucede en nuestra comunidad!

> **Web:** https://rustlang-es.org
> **Blog:** <https://rustlang-es.org/blog>
> **Recursos para aprender Rust:** https://rustlang-es.org/aprende
> **GitHub:** <https://github.com/RustLangES>
> **Linkedin:** <https://www.linkedin.com/company/rustlanges>

¡Bienvenidx una vez más y disfruta de tu estancia en nuestro servidor!"#;

pub async fn guild_member_addition(ctx: &Context, guild_id: &GuildId, member: &Member) {
    if let Err(e) = _guild_member_addition(ctx, guild_id, member).await {
        tracing::error!("Failed to handle welcome guild_member_addition: {}", e);
    }
}

#[tracing::instrument(skip(ctx))]
async fn _guild_member_addition(ctx: &Context, guild_id: &GuildId, member: &Member) -> Result<()> {
    let join_msg = "Bienvenid@ <mention> a <server>! Pásala lindo!".to_string();

    let msg_channel = ChannelId::new(778674893851983932_u64);

    let join_msg_replaced = join_msg
        .replace("<mention>", &member.user.mention().to_string())
        .replace("<username>", &member.user.name)
        .replace("<server>", &guild_id.name(ctx).unwrap_or_else(|| "".into()));

    // Download the user's avatar and create a welcome image
    let avatar_url = member
        .user
        .avatar_url()
        .unwrap_or_else(|| member.user.default_avatar_url());
    let response = reqwest::get(avatar_url).await?;
    let bytes = response.bytes().await?;

    let img = image::load_from_memory(&bytes)?;
    img.resize(256, 256, image::imageops::Lanczos3);
    let mut background = image::open("./static/background.png")?;

    let output_path = format!("/tmp/{}_welcome.png", member.user.name);
    combine_images(&mut background, &img, 74, 74, 372)?;
    background.save(output_path.as_str())?;
    let attachment = CreateAttachment::path(output_path.as_str()).await?;

    let msg = msg_channel
        .send_files(
            &ctx,
            vec![attachment],
            CreateMessage::new().content(&join_msg_replaced),
        )
        .await?;

    // Remove the file after sending the message
    std::fs::remove_file(&output_path)?;

    // Convert string emoji to ReactionType to allow custom emojis
    let reaction = ReactionType::try_from("👋")?;
    msg.react(&ctx, reaction).await?;

    // Send DM with guides
    member
        .user
        .dm(ctx, CreateMessage::new().content(WELCOME_MESSAGE))
        .await?;

    Ok(())
}

