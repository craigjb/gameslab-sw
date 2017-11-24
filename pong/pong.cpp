#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/fcntl.h>
#include <sys/epoll.h>
#include <unistd.h>
#include <linux/input.h>
#include <time.h>

#define SCREEN_W 800
#define SCREEN_H 480
#define PADDLE_W 8 
#define PADDLE_H 50
#define BALL_S 10
#define P1_X 20
#define P2_X (SCREEN_W - 20 - PADDLE_W)

int p1_y = 0;
int p2_y = SCREEN_H - PADDLE_H;
int ball_x = ((SCREEN_W / 2) - (BALL_S / 2));
int ball_y = ((SCREEN_H / 2) - (BALL_S / 2));
int ball_vx = -4;
int ball_vy = 4;

#define TOUCH_SLOTS 10
bool touch_active[TOUCH_SLOTS];
int touch_x[TOUCH_SLOTS];
int touch_y[TOUCH_SLOTS];

unsigned char* framebuffer = NULL;
unsigned char* backbuffer = NULL;
bool done = true;

int tsfd;

int imax(int num, int m) {
    return num > m ? m : num;
}

int imin(int num, int m) {
    return num < m ? m : num;
}

void clear()
{
    memset(backbuffer, 0x0, SCREEN_W * SCREEN_H * 3);
}

void flip()
{
    memcpy(framebuffer, backbuffer, SCREEN_W * SCREEN_H * 3);
}

void draw_paddle(int pos_x, int pos_y)
{
    for (int y = pos_y; y < pos_y + PADDLE_H; y++)
    {
        for (int x = pos_x; x < pos_x + PADDLE_W; x++)
        {
            backbuffer[(y * SCREEN_W * 3) + (x * 3)] = 0xFF;
            backbuffer[(y * SCREEN_W * 3) + (x * 3) + 1] = 0x0;
            backbuffer[(y * SCREEN_W * 3) + (x * 3) + 2] = 0x0;
        }
    }
}

void draw_ball(int pos_x, int pos_y)
{
    for (int y = pos_y; y < pos_y + BALL_S; y++)
    {
        for (int x = pos_x; x < pos_x + BALL_S; x++)
        {
            backbuffer[(y * SCREEN_W * 3) + (x * 3)] = 0xFF;
            backbuffer[(y * SCREEN_W * 3) + (x * 3) + 1] = 0xFF;
            backbuffer[(y * SCREEN_W * 3) + (x * 3) + 2] = 0xFF;
        }
    }
}

int slot = 0;
int x = -1;
int y = -1;
int active = -1;

void handle_input()
{
    struct input_event event;
    while (read(tsfd, &event, sizeof(struct input_event)) != -1)
    {
        if (event.code == ABS_MT_SLOT) {
            if (x >= 0) {
                touch_x[slot] = x;
            }
            if (y >= 0) {
                touch_y[slot] = y;
            }
            if (active >= 0) {
                touch_active[slot] = active;
            }
            x = -1;
            y = -1;
            active = -1;
            slot = event.value;
        } else if (event.code == SYN_REPORT) {
            if (x >= 0) {
                touch_x[slot] = x;
            }
            if (y >= 0) {
                touch_y[slot] = y;
            }
            if (active >= 0) {
                touch_active[slot] = active;
            }
            x = -1;
            y = -1;
            active = -1;
        } else if (event.code == ABS_MT_POSITION_X) {
            x = imax(imin(event.value, 0), SCREEN_W);
        } else if (event.code == ABS_MT_POSITION_Y) {
            y = imax(imin(event.value, 0), SCREEN_H - PADDLE_H);
        } else if (event.code == ABS_MT_TRACKING_ID) {
            if (event.value > 0) {
                active = 1;
                done = false;
            } else {
                active = 0;
            }
        }
    }

    for (int i = 0; i < TOUCH_SLOTS; i++) {
        if (touch_active[i]) {
            if (touch_x[i] < SCREEN_W / 2) {
                p1_y = imax(SCREEN_H - PADDLE_H, imin(0, touch_y[i] - (PADDLE_H / 2)));
            } else {
                p2_y = imax(SCREEN_H - PADDLE_H, imin(0, touch_y[i] - (PADDLE_H / 2)));
            }
        }
    }
}

void update_ball()
{
    if (!done)
    {
        if (!(ball_x < P1_X - PADDLE_W || ball_x > P1_X + PADDLE_W ||
                    ball_y < p1_y || ball_y > p1_y + PADDLE_H)) {
            if (ball_vx < 0) {
                ball_vx *= -1;
            }
        }

        if (!(ball_x < P2_X || ball_x > P2_X + (2 * PADDLE_W) ||
                    ball_y < p2_y || ball_y > p2_y + PADDLE_H)) {
            if (ball_vx > 0) {
                ball_vx *= -1;
            }
        }

        ball_x += ball_vx;
        if (ball_x < 0) {
            ball_x = 0;
            done = true;
            return;
        }
        if (ball_x > SCREEN_W - BALL_S) {
            ball_x = SCREEN_W - BALL_S;
            done = true;
            return;
        }
       
        ball_y += ball_vy;
        if (ball_y < 0) {
            ball_vy *= -1;
            ball_y -= ball_y;
        }

        if (ball_y > SCREEN_H - BALL_S) {
            ball_vy *= -1;
            ball_y -= ball_y - (SCREEN_H - BALL_S);
        }
    } else {
        ball_x = ((SCREEN_W / 2) - (BALL_S / 2));
        ball_y = ((SCREEN_H / 2) - (BALL_S / 2));
        if (rand() % 2 == 0) {
            ball_vx = 4;
        } else {
            ball_vx = -4;
        }
        if (rand() % 2 == 0) {
            ball_vy = 4;
        } else {
            ball_vy = -4;
        }
    }
}

void tick()
{
    clear();
    handle_input();
    update_ball();
    draw_paddle(P1_X, p1_y);
    draw_paddle(P2_X, p2_y);
    draw_ball(ball_x, ball_y);
    flip();
}

int main()
{
    int fbfd = open("/dev/fb0", O_RDWR);
    if (fbfd < 0) {
        printf("Error opening framebuffer!\n");
        return 1;
    }

    framebuffer = (unsigned char*)mmap(
        NULL, SCREEN_H * SCREEN_W * 3,
        PROT_READ | PROT_WRITE, MAP_SHARED, fbfd, 0);

    backbuffer = (unsigned char*)malloc(SCREEN_H * SCREEN_W * 3);

    tsfd = open("/dev/input/event0", O_RDONLY | O_NONBLOCK);
    if (tsfd < 0) {
        printf("Error opening touchscreen!\n");
        return 1;
    }

    struct timespec ts;
    ts.tv_sec = 0;
    ts.tv_nsec = 8 * 1000 * 1000;

    while (1) {
        tick();
        nanosleep(&ts, NULL);
    }

    close(fbfd);
    close(tsfd);

    return 0;
}
