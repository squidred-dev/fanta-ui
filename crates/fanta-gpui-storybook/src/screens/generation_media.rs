//! Storybook-owned media sessions for the generation gallery.
//!
//! The reusable generation screen receives this view as host content. Decoding,
//! sound output, player lifetime and media URLs remain responsibilities of a
//! host; none of these fixtures are part of `fanta-gpui`.

use std::{
    process::Child,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

use gpui::{
    Bounds, Context, Image, ImageFormat, InteractiveElement as _, MouseButton, MouseDownEvent,
    ObjectFit, ParentElement as _, Pixels, Point, Render, SharedString, Styled as _,
    StyledImage as _, Window, canvas, div, fill, img, point, prelude::FluentBuilder as _, px, size,
};
use gpui_component::{
    ActiveTheme as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

#[cfg(target_os = "macos")]
use core_video::pixel_buffer::CVPixelBuffer;
#[cfg(target_os = "macos")]
use gpui::{AppContext as _, Task, surface};
#[cfg(target_os = "macos")]
use media::video::{
    VideoFrameUpdate, VideoPlayback, VideoPlaybackState, VideoPlaybackStatus,
    prepare_video_playback, video_host_time_seconds,
};

const NATURE_POSTER: &[u8] = include_bytes!("../../assets/generation/video-nature.png");
const PRODUCT_POSTER: &[u8] = include_bytes!("../../assets/generation/product-study.png");
#[cfg(target_os = "macos")]
const NATURE_VIDEO: &[u8] = include_bytes!("../../assets/generation/video-nature-preview.mp4");
#[cfg(target_os = "macos")]
const PRODUCT_VIDEO: &[u8] = include_bytes!("../../assets/generation/video-product-preview.mp4");
const VOICE_ART: &[u8] = include_bytes!("../../assets/generation/audio-voice.svg");
const MUSIC_ART: &[u8] = include_bytes!("../../assets/generation/audio-music.svg");
#[cfg(target_os = "macos")]
const VOICE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/generation/audio-voice-preview.wav"
);
#[cfg(target_os = "macos")]
const MUSIC_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/generation/audio-music-preview.wav"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GenerationMediaClip {
    NatureVideo,
    ProductVideo,
    Voice,
    Music,
}

impl GenerationMediaClip {
    fn is_video(self) -> bool {
        matches!(self, Self::NatureVideo | Self::ProductVideo)
    }

    fn duration(self) -> Duration {
        match self {
            Self::NatureVideo | Self::ProductVideo => Duration::from_secs(5),
            Self::Voice => Duration::from_millis(3_794),
            Self::Music => Duration::from_millis(7_500),
        }
    }

    #[cfg(target_os = "macos")]
    fn audio_path(self) -> Option<&'static str> {
        match self {
            Self::NatureVideo | Self::ProductVideo => None,
            Self::Voice => Some(VOICE_PATH),
            Self::Music => Some(MUSIC_PATH),
        }
    }

    #[cfg(target_os = "macos")]
    fn video_bytes(self) -> Option<&'static [u8]> {
        match self {
            Self::NatureVideo => Some(NATURE_VIDEO),
            Self::ProductVideo => Some(PRODUCT_VIDEO),
            Self::Voice | Self::Music => None,
        }
    }

    fn poster(self) -> Arc<Image> {
        let (format, bytes) = match self {
            Self::NatureVideo => (ImageFormat::Png, NATURE_POSTER),
            Self::ProductVideo => (ImageFormat::Png, PRODUCT_POSTER),
            Self::Voice => (ImageFormat::Svg, VOICE_ART),
            Self::Music => (ImageFormat::Svg, MUSIC_ART),
        };
        Arc::new(Image::from_bytes(format, bytes.to_vec()))
    }
}

/// A real, Storybook-local player. A production host would resolve fresh
/// authenticated output bytes and provide its own GPUI view instead.
pub(crate) struct GenerationMediaView {
    clip: GenerationMediaClip,
    poster: Arc<Image>,
    #[cfg(target_os = "macos")]
    preparation: Option<Task<()>>,
    #[cfg(target_os = "macos")]
    video: Option<VideoPlayback>,
    #[cfg(target_os = "macos")]
    frame: Option<CVPixelBuffer>,
    #[cfg(target_os = "macos")]
    video_status: VideoPlaybackStatus,
    audio_process: Option<Child>,
    audio_started_at: Option<Instant>,
    audio_elapsed: Duration,
    playing: bool,
    muted: bool,
    frame_scheduled: bool,
    seek_bounds: Option<Bounds<Pixels>>,
    error: Option<SharedString>,
}

impl GenerationMediaView {
    pub(crate) fn new(clip: GenerationMediaClip, cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            clip,
            poster: clip.poster(),
            #[cfg(target_os = "macos")]
            preparation: None,
            #[cfg(target_os = "macos")]
            video: None,
            #[cfg(target_os = "macos")]
            frame: None,
            #[cfg(target_os = "macos")]
            video_status: VideoPlaybackStatus {
                state: VideoPlaybackState::Loading,
                current_time_us: 0,
                duration_us: 0,
            },
            audio_process: None,
            audio_started_at: None,
            audio_elapsed: Duration::ZERO,
            playing: false,
            muted: false,
            frame_scheduled: false,
            seek_bounds: None,
            error: None,
        };
        if clip.is_video() {
            view.prepare_video(cx);
        }
        view
    }

    #[cfg(target_os = "macos")]
    fn prepare_video(&mut self, cx: &mut Context<Self>) {
        let Some(bytes) = self.clip.video_bytes() else {
            return;
        };
        let work =
            cx.background_spawn(
                async move { prepare_video_playback(Arc::from(bytes), 1_200)?.await },
            );
        self.preparation = Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                this.preparation = None;
                match result.and_then(VideoPlayback::new) {
                    Ok(mut player) => {
                        if let Err(error) = player
                            .set_audio(this.muted, 0.8)
                            .and_then(|()| player.seek(0))
                        {
                            this.error = Some(format!("{error:#}").into());
                        } else {
                            if this.playing
                                && let Err(error) = player.play()
                            {
                                this.playing = false;
                                this.error = Some(format!("{error:#}").into());
                            }
                            this.video = Some(player);
                        }
                    }
                    Err(error) => this.error = Some(format!("{error:#}").into()),
                }
                cx.notify();
            });
        }));
    }

    #[cfg(not(target_os = "macos"))]
    fn prepare_video(&mut self, cx: &mut Context<Self>) {
        self.error = Some("Inline video playback is available on macOS.".into());
        cx.notify();
    }

    pub(crate) fn play(&mut self, cx: &mut Context<Self>) {
        if self.error.is_some() || self.playing {
            return;
        }
        match self.clip {
            GenerationMediaClip::NatureVideo | GenerationMediaClip::ProductVideo => {
                #[cfg(target_os = "macos")]
                if let Some(player) = &mut self.video {
                    if self.video_status.state == VideoPlaybackState::Ended
                        && let Err(error) = player.seek(0)
                    {
                        self.error = Some(format!("{error:#}").into());
                        return;
                    }
                    if let Err(error) = player.play() {
                        self.error = Some(format!("{error:#}").into());
                        return;
                    }
                }
                self.playing = true;
            }
            GenerationMediaClip::Voice | GenerationMediaClip::Music => {
                #[cfg(target_os = "macos")]
                {
                    if let Some(process) = &self.audio_process {
                        let result = Command::new("/bin/kill")
                            .arg("-CONT")
                            .arg(process.id().to_string())
                            .status();
                        if !result.is_ok_and(|status| status.success()) {
                            self.error = Some("Could not resume the audio preview.".into());
                            return;
                        }
                    } else {
                        let Some(path) = self.clip.audio_path() else {
                            return;
                        };
                        match Command::new("/usr/bin/afplay")
                            .arg(path)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                        {
                            Ok(process) => {
                                self.audio_process = Some(process);
                                self.audio_elapsed = Duration::ZERO;
                            }
                            Err(error) => {
                                self.error = Some(format!("Could not play audio: {error}").into());
                                return;
                            }
                        }
                    }
                    self.audio_started_at = Some(Instant::now());
                    self.playing = true;
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.error = Some("Storybook audio playback is available on macOS.".into());
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn pause(&mut self, cx: &mut Context<Self>) {
        if !self.playing {
            return;
        }
        match self.clip {
            GenerationMediaClip::NatureVideo | GenerationMediaClip::ProductVideo => {
                #[cfg(target_os = "macos")]
                if let Some(player) = &mut self.video
                    && let Err(error) = player.pause()
                {
                    self.error = Some(format!("{error:#}").into());
                }
            }
            GenerationMediaClip::Voice | GenerationMediaClip::Music => {
                if let Some(started) = self.audio_started_at.take() {
                    self.audio_elapsed += started.elapsed();
                }
                #[cfg(target_os = "macos")]
                if let Some(process) = &self.audio_process {
                    let result = Command::new("/bin/kill")
                        .arg("-STOP")
                        .arg(process.id().to_string())
                        .status();
                    if !result.is_ok_and(|status| status.success()) {
                        self.error = Some("Could not pause the audio preview.".into());
                    }
                }
            }
        }
        self.playing = false;
        cx.notify();
    }

    pub(crate) fn stop(&mut self, cx: &mut Context<Self>) {
        self.pause(cx);
        if let Some(mut process) = self.audio_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.audio_started_at = None;
        self.audio_elapsed = Duration::ZERO;
        #[cfg(target_os = "macos")]
        if let Some(player) = &mut self.video {
            let _ = player.seek(0);
        }
        cx.notify();
    }

    fn restart(&mut self, cx: &mut Context<Self>) {
        self.stop(cx);
        self.play(cx);
    }

    fn toggle_mute(&mut self, cx: &mut Context<Self>) {
        self.muted = !self.muted;
        #[cfg(target_os = "macos")]
        if let Some(player) = &mut self.video
            && let Err(error) = player.set_audio(self.muted, 0.8)
        {
            self.error = Some(format!("Could not change sound: {error:#}").into());
        }
        cx.notify();
    }

    fn elapsed(&self) -> Duration {
        match self.clip {
            GenerationMediaClip::NatureVideo | GenerationMediaClip::ProductVideo => {
                #[cfg(target_os = "macos")]
                {
                    Duration::from_micros(self.video_status.current_time_us)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Duration::ZERO
                }
            }
            GenerationMediaClip::Voice | GenerationMediaClip::Music => {
                self.audio_elapsed
                    + self
                        .audio_started_at
                        .map_or(Duration::ZERO, |start| start.elapsed())
            }
        }
    }

    fn duration(&self) -> Duration {
        #[cfg(target_os = "macos")]
        if self.clip.is_video() && self.video_status.duration_us > 0 {
            return Duration::from_micros(self.video_status.duration_us);
        }
        self.clip.duration()
    }

    fn seek_at(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        if let (Some(bounds), Some(player)) = (self.seek_bounds, self.video.as_mut()) {
            if bounds.size.width <= px(0.) || self.video_status.duration_us == 0 {
                return;
            }
            let ratio = ((position.x - bounds.origin.x) / bounds.size.width).clamp(0., 1.);
            let time_us = (self.video_status.duration_us as f64 * ratio as f64).round() as u64;
            match player.seek(time_us) {
                Ok(()) => {
                    self.video_status.current_time_us = time_us;
                    cx.notify();
                }
                Err(error) => self.error = Some(format!("Could not seek: {error:#}").into()),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (position, cx);
        }
    }

    fn seek_by(&mut self, seconds: i64, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        if let Some(player) = &mut self.video {
            let current = self.video_status.current_time_us as i64;
            let duration = self.video_status.duration_us as i64;
            let target = current
                .saturating_add(seconds.saturating_mul(1_000_000))
                .clamp(0, duration) as u64;
            match player.seek(target) {
                Ok(()) => {
                    self.video_status.current_time_us = target;
                    cx.notify();
                }
                Err(error) => self.error = Some(format!("Could not seek: {error:#}").into()),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (seconds, cx);
        }
    }

    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        if let Some(player) = &mut self.video {
            let result = player.status().and_then(|status| {
                let frame = player.frame_for_host_time(video_host_time_seconds())?;
                Ok((status, frame))
            });
            match result {
                Ok((status, frame)) => {
                    let changed = status != self.video_status;
                    self.video_status = status;
                    if status.state == VideoPlaybackState::Ended {
                        self.playing = false;
                    }
                    let changed_frame = match frame {
                        VideoFrameUpdate::Frame(frame) => {
                            self.frame = Some(frame.buffer);
                            true
                        }
                        VideoFrameUpdate::Empty => {
                            self.frame = None;
                            true
                        }
                        VideoFrameUpdate::Unchanged => false,
                    };
                    if changed || changed_frame {
                        cx.notify();
                    }
                }
                Err(error) => {
                    self.video = None;
                    self.playing = false;
                    self.error = Some(format!("{error:#}").into());
                    cx.notify();
                }
            }
        }

        if let Some(process) = &mut self.audio_process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    self.audio_process = None;
                    self.audio_started_at = None;
                    self.playing = false;
                    if status.success() {
                        self.audio_elapsed = self.duration();
                    } else {
                        self.error = Some("The audio preview could not be played.".into());
                    }
                    cx.notify();
                }
                Ok(None) => {}
                Err(error) => {
                    self.audio_process = None;
                    self.audio_started_at = None;
                    self.playing = false;
                    self.error = Some(format!("Audio playback failed: {error}").into());
                    cx.notify();
                }
            }
        }

        let waiting_video = {
            #[cfg(target_os = "macos")]
            {
                self.preparation.is_some()
                    || matches!(
                        self.video_status.state,
                        VideoPlaybackState::Loading | VideoPlaybackState::Seeking
                    ) && self.video.is_some()
            }
            #[cfg(not(target_os = "macos"))]
            {
                false
            }
        };
        if !self.frame_scheduled && (self.playing || waiting_video) {
            self.frame_scheduled = true;
            let weak = cx.entity().downgrade();
            window.on_next_frame(move |window, cx| {
                if let Some(view) = weak.upgrade() {
                    view.update(cx, |this, cx| {
                        this.frame_scheduled = false;
                        this.tick(window, cx);
                    });
                }
            });
        }
    }
}

impl Drop for GenerationMediaView {
    fn drop(&mut self) {
        if let Some(mut process) = self.audio_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl Render for GenerationMediaView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.tick(window, cx);
        let elapsed = self.elapsed().min(self.duration());
        let duration = self.duration();
        let ratio = if duration.is_zero() {
            0.
        } else {
            elapsed.as_secs_f32() / duration.as_secs_f32()
        };
        let colors = cx.theme();
        let foreground = colors.foreground;
        let muted = colors.muted_foreground;
        let background = colors.background;
        let border = colors.border;
        let progress = colors.primary;
        let is_video = self.clip.is_video();
        let mut waveform = h_flex()
            .items_center()
            .justify_center()
            .gap(px(3.))
            .h(px(54.));
        for index in 0..56 {
            let wave = ((index as f32 * 0.41).sin().abs() * 23.
                + (index as f32 * 0.17).cos().abs() * 15.
                + 6.)
                .min(50.);
            waveform = waveform.child(div().w(px(3.)).h(px(wave)).rounded_full().bg(
                if index as f32 / 56. <= ratio {
                    progress
                } else {
                    muted.opacity(0.42)
                },
            ));
        }
        let artwork = div()
            .flex_1()
            .min_h_0()
            .w_full()
            .overflow_hidden()
            .bg(background)
            .flex()
            .items_center()
            .justify_center()
            .when(is_video, |element| {
                #[cfg(target_os = "macos")]
                if let Some(frame) = self.frame.clone() {
                    return element.child(surface(frame).size_full());
                }
                element.child(
                    img(self.poster.clone())
                        .size_full()
                        .object_fit(ObjectFit::Contain),
                )
            })
            .when(!is_video, |element| {
                element.child(
                    v_flex()
                        .w_full()
                        .items_center()
                        .justify_center()
                        .gap(px(20.))
                        .p(px(24.))
                        .child(
                            img(self.poster.clone())
                                .w(px(220.))
                                .h(px(220.))
                                .object_fit(ObjectFit::Contain),
                        )
                        .child(waveform)
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(muted)
                                .child("AUDIO PREVIEW"),
                        ),
                )
            });
        let seek_view = cx.entity().downgrade();
        let seek_bar = div()
            .id("generation-media-seek")
            .w_full()
            .h(px(20.))
            .when(is_video, |element| {
                element.cursor_pointer().on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &MouseDownEvent, _, cx| {
                        this.seek_at(event.position, cx);
                        cx.stop_propagation();
                    }),
                )
            })
            .child(
                canvas(
                    move |bounds, _, cx| {
                        if let Some(view) = seek_view.upgrade() {
                            view.update(cx, |this, _| this.seek_bounds = Some(bounds));
                        }
                    },
                    move |bounds, _, window, _cx| {
                        let track = Bounds {
                            origin: point(bounds.origin.x, bounds.center().y - px(2.)),
                            size: size(bounds.size.width, px(4.)),
                        };
                        window.paint_quad(fill(track, border).corner_radii(px(2.)));
                        let played = Bounds {
                            size: size(track.size.width * ratio.clamp(0., 1.), track.size.height),
                            ..track
                        };
                        window.paint_quad(fill(played, progress).corner_radii(px(2.)));
                        let thumb = Bounds {
                            origin: point(played.right() - px(5.), bounds.center().y - px(5.)),
                            size: size(px(10.), px(10.)),
                        };
                        window.paint_quad(fill(thumb, progress).corner_radii(px(5.)));
                    },
                )
                .size_full(),
            );
        let mut controls = h_flex()
            .items_center()
            .gap(px(8.))
            .child(
                Button::new("generation-media-play-toggle")
                    .label(if self.playing { "Pause" } else { "Play" })
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        if this.playing {
                            this.pause(cx);
                        } else {
                            this.play(cx);
                        }
                    })),
            )
            .child(
                Button::new("generation-media-restart")
                    .label("Restart")
                    .ghost()
                    .compact()
                    .on_click(cx.listener(|this, _, _, cx| this.restart(cx))),
            );
        if is_video {
            controls = controls
                .child(
                    Button::new("generation-media-backward")
                        .label("−5 s")
                        .ghost()
                        .compact()
                        .on_click(cx.listener(|this, _, _, cx| this.seek_by(-5, cx))),
                )
                .child(
                    Button::new("generation-media-forward")
                        .label("+5 s")
                        .ghost()
                        .compact()
                        .on_click(cx.listener(|this, _, _, cx| this.seek_by(5, cx))),
                );
        }
        let transport = v_flex()
            .w_full()
            .p(px(16.))
            .gap(px(8.))
            .bg(background)
            .border_t_1()
            .border_color(border)
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(muted)
                            .child(if is_video {
                                "VIDEO PREVIEW"
                            } else {
                                "AUDIO PREVIEW"
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(foreground)
                            .child(format!(
                                "{} / {}",
                                time_label(elapsed),
                                time_label(duration)
                            )),
                    ),
            )
            .child(seek_bar)
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .child(controls)
                    .when(is_video, |row| {
                        row.child(
                            Button::new("generation-media-sound")
                                .label(if self.muted { "Unmute" } else { "Mute" })
                                .ghost()
                                .compact()
                                .on_click(cx.listener(|this, _, _, cx| this.toggle_mute(cx))),
                        )
                    }),
            )
            .when_some(self.error.clone(), |element, error| {
                element.child(
                    div()
                        .text_size(px(11.))
                        .text_color(colors.danger)
                        .child(error),
                )
            });

        v_flex()
            .size_full()
            .min_h_0()
            .child(artwork)
            .child(transport)
    }
}

fn time_label(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
